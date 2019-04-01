#!/usr/bin/env ruby

require "childprocess"
require "toml"

ARGV.first&.split(".") || []

class String
  def int?
    /^\d+$/.match?(self)
  end
end

DockerError = Class.new(StandardError)

class InvalidVersion < StandardError
  def initialize
    super("Requires version of format x.y.z")
  end
end

def validate_version(version)
  semver = version&.split(".")
  raise InvalidVersion unless semver&.size == 3
  raise InvalidVersion unless semver&.all?(&:int?)
  return semver.map.with_index { |v, i| semver[0..i].join(".") }
end

def docker(*args)
  process = ChildProcess.build("docker", *args).tap do |p|
    p.io.inherit!
    p.start
    p.wait
  end

  raise DockerError unless process.exit_code.zero?
end

def docker_build(image_name, versions)
  tags = versions.flat_map { |v| ["-t", "#{image_name}:#{v}"] }
  puts "Building #{versions.size} images..."
  docker "build", *tags, "."
end

def docker_push(image_name, versions)
  puts "Pushing #{versions.size} images to #{image_name}"
  versions.each do |version|
    docker "push", "#{image_name}:#{version}"
  end
end

version = TOML.load_file("Cargo.toml").dig("package", "version")
image_name = "registry.gitlab.com/valeth/javelin"

begin
  versions = ["latest", *validate_version(version)]
  docker_build(image_name, versions)
  docker_push(image_name, versions)
rescue InvalidVersion, DockerError => e
  warn e.message
  exit 1
end

